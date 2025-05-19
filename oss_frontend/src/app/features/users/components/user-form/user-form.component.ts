import { Component, OnInit } from '@angular/core';
import {FormBuilder, FormGroup, FormsModule, ReactiveFormsModule, Validators} from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { forkJoin, Observable, of } from 'rxjs';
import { catchError, finalize, switchMap } from 'rxjs/operators';
import {TranslatePipe, TranslateService} from '@ngx-translate/core';
import { UserGroup } from "../../../../shared/models/user-group.model";
import { UserIdentifierType } from "../../../../shared/models/user-identifier-type.model";
import { UserIdentifier } from "../../../../shared/models/user-identifier.model";
import { UserService } from "../../../../shared/services/user.service";
import { UserIdentifierTypeService } from "../../../../shared/services/user-identifier-type.service";
import { AttributeGroup } from 'src/app/shared/models/attribute-group.model';
import { AttributeGroupService } from 'src/app/shared/services/attribute-group.service';

interface LoadDataResult {
  groups: UserGroup[];
  identifierTypes: UserIdentifierType[];
  authAttributeGroups: AttributeGroup[];
}

interface UserResponse {
  email: string;
  external_id: string;
  first_name: string;
  last_name: string;
  phone_number: string;
  is_active: boolean;
  groups: UserGroup[];
  identifiers: UserIdentifier[];
}

interface EditModeResult extends LoadDataResult {
  user: UserResponse;
}

@Component({
  selector: 'app-user-form',
  templateUrl: './user-form.component.html',
  imports: [
    TranslatePipe,
    ReactiveFormsModule,
    FormsModule
  ],
  styleUrls: ['./user-form.component.scss']
})
export class UserFormComponent implements OnInit {
  userForm: FormGroup;
  isEditMode = false;
  userId: number | null = null;
  userGroups: UserGroup[] = [];
  identifierTypes: UserIdentifierType[] = [];
  authAttributeGroups: AttributeGroup[] = [];
  identifiers: Partial<UserIdentifier>[] = [];
  loading = true;
  submitting = false;
  error: string | null = null;

  constructor(
    private fb: FormBuilder,
    private userService: UserService,
    private identifierTypeService: UserIdentifierTypeService,
    private authAttributeGroupService:AttributeGroupService,
    private route: ActivatedRoute,
    private router: Router,
    private translate: TranslateService
  ) {
    this.userForm = this.fb.group({
      email: ['', [Validators.required, Validators.email]],
      first_name: [''],
      last_name: [''],
      external_id: [null],
      phone_number: [''],
      is_active: [true],
      groups: [[]]
    });
  }

  ngOnInit(): void {
    this.loading = true;
    this.error = null;

    const loadData$: Observable<LoadDataResult> = forkJoin({
      groups: this.userService.getUserGroupList(),
      identifierTypes: this.identifierTypeService.getIdentifierTypes(),
      authAttributeGroups: this.authAttributeGroupService.getAllGroupsList()
    });

    this.route.params.pipe(
      switchMap(params => {
        if (params['id']) {
          this.isEditMode = true;
          this.userId = +params['id'];
          return forkJoin({
            user: this.userService.getUser(this.userId),
            groups: this.userService.getUserGroupList(),
            identifierTypes: this.identifierTypeService.getIdentifierTypes(),
            authAttributeGroups: this.authAttributeGroupService.getAllGroupsList()
          }) as Observable<EditModeResult>;
        }
        return loadData$;
      }),
      catchError(error => {
        this.error = error.message || 'An error occurred while loading data';
        this.loading = false;
        return of(null);
      })
    ).subscribe({
      next: (result) => {
        if (result) {
          if (this.isEditMode && 'user' in result) {
            const editResult = result as EditModeResult;
            const userData = {
              email: editResult.user.email,
              first_name: editResult.user.first_name,
              last_name: editResult.user.last_name,
              phone_number: editResult.user.phone_number,
              is_active: editResult.user.is_active,
              groups: editResult.user.groups.map(g => g.id)
            };
            this.userForm.patchValue(userData);
            this.identifiers = editResult.user.identifiers || [];
          }
          this.userGroups = result.groups;
          this.identifierTypes = result.identifierTypes;
          this.authAttributeGroups = result.authAttributeGroups;
        }
        this.loading = false;
      },
      error: (error) => {
        this.error = error.message || 'An error occurred while loading data';
        this.loading = false;
      }
    });
  }

  onGroupChange(event: Event, group: UserGroup): void {
    const checkbox = event.target as HTMLInputElement;
    const groups = this.userForm.get('groups')?.value || [];

    if (checkbox.checked) {
      groups.push(group.id);
    } else {
      const index = groups.indexOf(group.id);
      if (index > -1) {
        groups.splice(index, 1);
      }
    }

    this.userForm.patchValue({ groups });
  }

  addIdentifier(): void {
    this.identifiers.push({
      identifier_type: this.identifierTypes[0],
      value: '',
      is_enabled: true,
      reject_expired: false,
      auth_attribute_group: null,
      expired_auth_attribute_group: null,
      expiration_date: null,
      comment: ''
    });
  }

  removeIdentifier(index: number): void {
    this.identifiers.splice(index, 1);
  }

  onSubmit(): void {
    if (this.userForm.invalid) {
      return;
    }

    this.submitting = true;
    this.error = null;

    const userData = {
      ...this.userForm.value,
      identifiers: this.identifiers
    };

    const request$ = this.isEditMode
      ? this.userService.updateUser(this.userId!, userData)
      : this.userService.createUser(userData);

    request$.pipe(
      catchError(error => {
        this.error = error.message || 'An error occurred while saving the user';
        return of(null);
      }),
      finalize(() => {
        this.submitting = false;
      })
    ).subscribe(result => {
      if (result) {
        this.router.navigate(['/users/users']);
      }
    });
  }

  goBack(): void {
    this.router.navigate(['/users/users']);
  }
}
